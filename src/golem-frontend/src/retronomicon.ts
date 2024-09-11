import type * as retronomicon from "@:retronomicon";
import * as net from "@:golem/net";

export type CoreList =
  retronomicon.paths["/cores"]["get"]["responses"]["200"]["content"]["application/json"];

export type ReleaseList =
  retronomicon.paths["/cores/{core_id}/releases"]["get"]["responses"]["200"]["content"]["application/json"];

export function cores(): CoreList {
  return net.fetchJson<CoreList>("https://retronomicon.land/api/v1/cores");
}

export function releases(coreId: number): ReleaseList {
  return net.fetchJson<ReleaseList>(
    `https://retronomicon.land/api/v1/cores/${coreId}/releases`,
  );
}
