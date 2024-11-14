export default function (constants) {
  return {
    name: "1fpga-consts",
    async load(id) {
      if (id.startsWith("consts:")) {
        const name = id.slice(7);
        const output = constants[name];

        return `
          export default ${JSON.stringify(output)};
        `;
      }
      return null;
    },
    resolveId(source) {
      if (source.startsWith("consts:")) {
        return source;
      }
      return null;
    },
  };
}
