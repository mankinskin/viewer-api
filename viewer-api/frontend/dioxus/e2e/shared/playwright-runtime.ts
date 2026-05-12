import { createRequire } from 'node:module';

const requireFromRunner = createRequire(`${process.cwd()}/package.json`);

export const { test, expect } = requireFromRunner('@playwright/test') as typeof import('@playwright/test');