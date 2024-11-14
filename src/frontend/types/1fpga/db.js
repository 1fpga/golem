// This file is a placeholder used in tests. Rollup should skip this file.
((...names) => {
  names.forEach((name) => {
    module.exports[name] = function () {
      throw `'db.${name}' - This should not end up in any production code.`;
    };
  });
})("load");
