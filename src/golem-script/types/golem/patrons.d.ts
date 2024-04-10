// Patreon's members and code contributors information.

export interface Patron {
    tiers: {
        [amount: string]: string;
    }
    patrons: {
        [tier: string]: string[];
    }
}

declare const PATRONS: Patron[];

declare module "golem/patrons" {
    export default PATRONS;
}
