import { Base64 } from "js-base64";
import * as fs from "@:golem/fs";
import * as ui from "@:golem/ui";
import * as upgrade from "@:golem/upgrade";
import * as net from "@:golem/net";
import type {
  Binary as ReleaseBinary,
  Release as ReleasesReleaseSchema,
  Releases as ReleasesSchema,
} from "$schemas:catalog/releases";
import { compareVersions, fetchJsonAndValidate } from "$/utils";
import { RemoteCatalog } from "$/services";
import { stripIndents } from "common-tags";

export const enum KnownBinary {
  OneFpga = "1fpga",
}

/**
 * A list of all releases from the `releases.json` file.
 */
export class RemoteReleases {
  public static async fetch(url: string, catalog: RemoteCatalog) {
    const releasesUrl = new URL(url, catalog.url).toString();
    const releases = await fetchJsonAndValidate(
      releasesUrl,
      (await import("$schemas:catalog/releases")).validate,
    );
    return new RemoteReleases(releasesUrl, releases, catalog);
  }

  constructor(
    public readonly url: string,
    public readonly schema: ReleasesSchema,
    public readonly catalog: RemoteCatalog,
  ) {}

  public asObject(predicate: (name: string) => boolean = () => true) {
    return Object.fromEntries(
      Object.entries(this.schema)
        .filter(([name]) => predicate(name))
        .map(([name, release]) => [
          name,
          new RemoteBinary(name, release, this),
        ]),
    );
  }

  public binaryNamed(name: string) {
    if (Object.getOwnPropertyNames(this.schema).includes(name)) {
      return new RemoteBinary(name, this.schema[name], this);
    } else {
      return undefined;
    }
  }
}

/**
 * A binary that's part of a release.
 */
export class RemoteBinary {
  constructor(
    public readonly name: string,
    private readonly schema_: ReleaseBinary,
    private readonly releases_: RemoteReleases,
  ) {}

  public get url() {
    return this.releases_.url;
  }

  public latestVersion(): RemoteRelease | undefined {
    let highestVersion;
    for (const release of this.schema_) {
      if (release.tags?.includes("latest")) {
        return new RemoteRelease(release, this);
      }

      if (!highestVersion) {
        highestVersion = release;
      } else {
        if (compareVersions(highestVersion.version, release.version) > 0) {
          highestVersion = release;
        }
      }
    }

    return highestVersion ? new RemoteRelease(highestVersion, this) : undefined;
  }

  /**
   * Get the installed version of this binary.
   */
  public async getInstalledVersion(): Promise<string | undefined> {
    switch (this.name) {
      case KnownBinary.OneFpga: {
        const v = ONE_FPGA.version;
        return `${v.major}.${v.minor}.${v.patch}`;
      }
    }
  }
}

export class RemoteRelease {
  constructor(
    private readonly schema_: ReleasesReleaseSchema,
    private readonly binary_: RemoteBinary,
  ) {}

  public get version() {
    return this.schema_.version;
  }

  /**
   * Download the release and perform the upgrade.
   *
   * @param force If true, force the upgrade regardless of version.
   * @returns True if the upgrade was successful, false if there was a problem.
   */
  public async doUpgrade(force = false): Promise<boolean> {
    ui.show(
      `Downloading ${this.binary_.name}...`,
      `Please wait while the upgrade is performed.`,
    );

    const downloads: [string, Uint8Array | undefined][] = await Promise.all(
      this.schema_.files.map(async (f) => {
        const url = new URL(f.url, this.binary_.url).toString();
        const path = await net.download(url);
        let signature;
        if (!force) {
          if ((await fs.fileSize(path)) !== f.size) {
            throw new Error(`File size mismatch for ${f.url}`);
          }
          if ((await fs.sha256(path)) !== f.sha256) {
            throw new Error(`SHA-256 hash mismatch for ${f.url}`);
          }

          // If there is a signature it's in base64, deserialize and verify it.
          if (f.signature) {
            signature = Base64.toUint8Array(f.signature);
            const valid = await upgrade.verifySignature(path, signature);
            if (!valid) {
              throw new Error(`Invalid signature for ${f.url}`);
            }
          }
        } else {
          signature = undefined;
        }

        return [path, signature];
      }),
    );

    if (downloads.length !== 1) {
      throw new Error("Expected exactly one file to upgrade");
    }

    const [path, signature] = downloads[0];
    ui.show(
      `Upgrading ${this.binary_.name}...`,
      stripIndents`
        Please wait while the upgrade is performed.
        
        Do not power off or restart your device.
        
        The system will restart automatically after the upgrade is completed.
      `,
    );

    // This may not return.
    await upgrade.upgrade(this.binary_.name, path, signature);

    return true;
  }
}
