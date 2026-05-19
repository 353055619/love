import init, { start } from "./pkg/love.js";

async function boot() {
  await init();
  start();
}

boot();
