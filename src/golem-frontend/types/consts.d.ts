declare module "consts:production" {
  /**
   * Whether the application is in production mode.
   */
  const production: boolean;
  export default production;
}

declare module "consts:*" {
  /**
   * Constant that will be inlined by Rollup and rollup-plugin-consts.
   */
  const constant: any;
  export default constant;
}
