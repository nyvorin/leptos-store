import { Container, getContainer } from "@cloudflare/containers";

export class ShowcaseContainer extends Container {
  defaultPort = 8080;
  sleepAfter = "30m";
}

export default {
  async fetch(request, env) {
    // Single shared instance — all visitors see the same showcase
    const container = getContainer(env.SHOWCASE, "showcase");
    return container.fetch(request);
  },
};
