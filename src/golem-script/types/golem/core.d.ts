// Type definitions for the `golem/core` module in Golem Script.

/**
 * This module provides functions to interact with cores in Golem Script.
 */
declare module "golem/core" {
    /**
     * A path to a core file.
     */
    export interface CorePath {
        type: "path",
        path: string,
    }

    /**
     * The type of core to start.
     */
    export type CoreType = CorePath;

    /**
     * A path to a game ROM.
     */
    export interface GameRomPath {
        type: "rom-path",
        path: string,
    }

    /**
     * The type of game to load.
     */
    export type GameType = GameRomPath;

    /**
     * Options for running a core.
     */
    export interface RunOptions {
        /**
         * The core to run.
         */
        core: CoreType,

        /**
         * A game to load.
         */
        game?: GameType,

        /**
         * The save file path to load (or save to). If missing the core will
         * not use any save file.
         */
        files?: [string?],

        /**
         * The savestate file to load. If savestates are not supported, this will be ignored.
         */
        savestate?: string,

        /**
         * Whether to show the core menu or not when launching. True by default.
         */
        showmenu?: boolean,

        /**
         * Whether to start a loop or to load the core and return it. Default
         * to true.
         */
        autoloop?: boolean,
    }

    export interface GolemCore {
        /**
         * The core's main loop. This function will return when the core is
         * unloaded by the user.
         */
        loop(): void;

        /**
         * Take a screenshot. Output the screenshot to the given path.
         * This is a blocking operation.
         */
        screenshot(path: string): void;
    }

    /**
     * Starts a core with the given options.
     * @param options The options for the core.
     */
    export function run(options: RunOptions): GolemCore | void;
}
