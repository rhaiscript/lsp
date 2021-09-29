/**
 * Asserts that the code is unreachable,
 * and will throw if ever reached.
 *
 * @param message Optional message.
 */
export function unreachable(message?: string): never {
  if (typeof message !== "undefined") {
    throw new Error(`unreachable code: ${message}`);
  }

  throw new Error(`unreachable code`);
}

/**
 * Asserts that a value will always be a never type.
 *
 * Useful for checking enum exhaustiveness.
 *
 * It also throws if this code is still somehow reached.
 *
 * @param _x The type that must be unreachable.
 * @param message Optional message.
 */
export function assertNever(_x: never, message?: string): never {
  if (typeof message !== "undefined") {
    throw new Error(`unreachable code: ${message}`);
  }

  throw new Error(`unreachable code`);
}

export const sleep = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms));
