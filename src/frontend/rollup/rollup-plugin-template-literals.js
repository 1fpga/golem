import babelParser from "@babel/parser";
import generate from "@babel/generator";
import traverse from "@babel/traverse";
import * as commonTags from "common-tags";

const { parse, ParserOptions } = babelParser;

/**
 * @param {string} code
 * @returns {string}
 */
function defaultTransformer(code) {
  return code;
}

/**
 * @typedef {Object} TransformerOptions
 * @property {ParserOptions} [parserOptions={}] - Parser options for Babel
 * @property {string[]} [tagsToProcess=[]] - List of named template tags to process
 * @property {defaultTransformer} [transformer] - Callback function for handling piece of code
 */

/**
 * @param {string} content
 * @param {TransformerOptions} [options={}]
 * @returns {string}
 */
export function transformTaggedContent(content, options = {}) {
  const {
    parserOptions = { sourceType: "module" },
    tagsToProcess = [],
    transformer = defaultTransformer,
  } = options;

  const ast = parse(content, parserOptions);

  traverse.default(ast, {
    TaggedTemplateExpression(path) {
      if (tagsToProcess.includes(path.node.tag.name)) {
        for (const quasi of path.node.quasi.quasis) {
          const transformedData = transformer(quasi.value.raw);
          quasi.value.raw = transformedData;
          quasi.value.cooked = transformedData;
        }
      }
    },
  });

  return generate.default(ast);
}

/**
 * @type {rollup.Plugin}
 * @param {TransformerOptions} [options={}]
 */
export function transformTaggedTemplate(options = {}) {
  return {
    name: "transform-tagged-template",
    transform(content, id) {
      if (id.endsWith(".json")) {
        return content;
      } else {
        return transformTaggedContent(content, options);
      }
    },
  };
}

/**
 * @type {rollup.Plugin}
 * @param {string} name The name of the function to use from `common-tags`
 */
export function transformCommonTags(name) {
  return {
    name: "transform-common-tags-" + name,
    transform(content, id) {
      if (id.endsWith(".json")) {
        return content;
      } else {
        return transformTaggedContent(content, {
          tagsToProcess: [name],
          transformer: (content, o) => {
            if (name === "oneLine") {
              return content.replace(/\n\s*/gm, " ").replace(/\s+/g, " ");
            } else {
              return commonTags[name](content);
            }
          },
        });
      }
    },
  };
}
