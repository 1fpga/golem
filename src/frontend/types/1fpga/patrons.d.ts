// Patreon's members and code contributors information.

declare const PATRONS: {
  tiers: {
    [amount: string]: string;
  };
  patrons: {
    [tier: string]: string[];
  };
};

declare module "1fpga:patrons" {
  export default PATRONS;
}
