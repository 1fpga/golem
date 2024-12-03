import * as osd from "1fpga:osd";

export class WizardCancelError {}

export class WizardCancelDone {}

export class WizardRestart {}

export interface StepOptions {
  previous: () => Promise<void>;
}

export type WizardStep<T> = {
  // The number of steps this step will take. By default, 1.
  count?: number;
  // The function to run the step.
  (options: StepOptions): Promise<T>;
};

export function map<T, U>(
  step: WizardStep<T>,
  next: (value: T) => Promise<U>,
): WizardStep<U> {
  return async (options: StepOptions) => {
    return await next(await step(options));
  };
}

export function ignore<T>(step: WizardStep<T>): WizardStep<undefined> {
  return async (options: StepOptions) => {
    await step(options);
    return undefined;
  };
}

export function conditional<T>(
  condition: WizardStep<boolean | undefined>,
  step: WizardStep<T>,
): WizardStep<T | undefined> {
  return async (options: StepOptions) => {
    if ((await condition(options)) === true) {
      return await step(options);
    }
  };
}

export function value<T>(value: T | Promise<T>): WizardStep<T> {
  return async (options) => await value;
}

export function call<T>(fn: () => Promise<T>): WizardStep<T> {
  return async (options) => await fn();
}

export function noop(): WizardStep<undefined> {
  return async (options) => {};
}

export function generate<T>(
  fn: () => Promise<WizardStep<T | undefined>>,
): WizardStep<T>;
export function generate<T>(
  fn: () => Promise<WizardStep<T | undefined>[]>,
): WizardStep<T[]>;

/**
 * Generate a step from a function that returns a list of steps, at the time
 * the wizard is running. If this step is skipped, the function will not be
 * called and those steps will not be shown.
 * @param fn
 */
export function generate<T>(
  fn: () => Promise<WizardStep<T | undefined> | WizardStep<T | undefined>[]>,
): WizardStep<T | T[] | undefined> {
  return async (options) => {
    const steps = await fn();

    if (Array.isArray(steps)) {
      return (await sequence(...steps)(options)).filter((x) => x !== undefined);
    } else {
      return (await sequence(steps)(options))[0];
    }
  };
}

export function skipIf<T>(
  condition: (options: StepOptions) => Promise<boolean>,
  step: WizardStep<T>,
  defaultValue?: T | undefined,
): WizardStep<T | undefined> {
  return async (options: StepOptions) => {
    if (!(await condition(options))) {
      return await step(options);
    } else {
      return defaultValue;
    }
  };
}

export function repeat<T>(
  condition: (lastResult: T) => Promise<boolean>,
  step: WizardStep<T>,
): WizardStep<T | undefined> {
  return async (options: StepOptions) => {
    let done = false;
    let result;
    do {
      result = await step({
        ...options,
        previous: async () => {
          done = true;
          await options.previous();
        },
      });
    } while (!done && (await condition(result)));
    return result;
  };
}

export function first<T>(
  step: WizardStep<T[] | undefined>,
): WizardStep<T | undefined> {
  return async (options: StepOptions) => {
    let result = await step(options);
    return result && result[0];
  };
}

export function last<T>(
  step: WizardStep<T[] | undefined>,
): WizardStep<T | undefined> {
  return async (options: StepOptions) => {
    let result = await step(options);
    return result && result[result.length - 1];
  };
}

export function sequence<T>(...steps: WizardStep<T>[]): WizardStep<T[]> {
  const fn = async (options: StepOptions) => {
    let done = false;
    let result: T[] = [];
    for (let i = 0; !done && i < steps.length; i++) {
      result.push(
        await steps[i]({
          ...options,
          previous: async () => {
            if (i == 0) {
              done = true;
              result = [];
              await options.previous();
            } else {
              i -= 2;
              result.pop();
            }
          },
        }),
      );
    }

    return result;
  };
  fn.count = steps.reduce((acc, step) => acc + (step.count ?? 1), 0);
  return fn;
}

export interface MessageOptions<T> {
  previous?: string;
  choices?: string[];
  map?: (choice: number) => T | undefined;

  /**
   * Allow user to cancel or not. If `true`, the user cannot cancel.
   */
  noCancel?: boolean;
}

export function message<T = number>(
  title: string,
  message: string,
  options?: MessageOptions<T>,
): WizardStep<T | undefined> {
  const choices = options?.choices ?? ["OK"];
  const noCancel = options?.noCancel ?? false;
  let previous = -1;
  if (options?.previous) {
    if (!choices.includes(options.previous)) {
      choices.unshift(options.previous);
    }
    previous = choices.indexOf(options.previous);
  }
  const mapper = options?.map ?? ((x) => x as unknown as T);

  return async (options: StepOptions) => {
    while (true) {
      const result = await osd.alert({
        title: `${title}`,
        message,
        choices,
      });

      // `noCancel` means the user cannot cancel. `osd.alert()` always
      // allows to cancel, so in this case we just ignore the result
      // and re-ask.
      if (noCancel && result === null) {
        continue;
      }

      if (result === null || result === previous) {
        await options.previous();
        return mapper(-1);
      } else {
        return mapper(result);
      }
    }
  };
}

export function choice<T>(
  title: string,
  message: string,
  choices: [string, WizardStep<T>][],
): WizardStep<T | undefined> {
  const fn = async (options: StepOptions) => {
    let done = false;
    let stepResult: T | undefined;
    while (!done) {
      const result = await osd.alert({
        title,
        message,
        choices: choices.map((c) => c[0]),
      });
      if (result === null) {
        await options.previous();
        return;
      }
      const choice = choices[result][1];
      done = true;
      stepResult = await choice({
        ...options,
        previous: async () => {
          done = false;
        },
      });
    }

    return stepResult;
  };
  fn.count = Math.max.apply(
    null,
    choices.map((c) => c[1].count ?? 1),
  );
  return fn;
}

export async function wizard<T>(
  steps: WizardStep<T>[],
  onError?: (e: any) => Promise<void>,
): Promise<void> {
  let seq = sequence(...steps);
  let options: StepOptions = {
    async previous(): Promise<void> {},
  };

  try {
    await seq(options);
  } catch (e) {
    // Swallow errors that are of known types.
    if (e instanceof WizardCancelError || e instanceof WizardCancelDone) {
    } else if (onError) {
      await onError(e);
    } else {
      throw e;
    }
  }
}
