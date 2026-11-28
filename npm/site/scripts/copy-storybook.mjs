import { access, cp } from "node:fs/promises";
import routes from "../src/data/site-routes.json" with { type: "json" };

const from = new URL("../../report-ui/storybook-static/", import.meta.url);
const to = new URL("../dist" + routes.storybook + "/", import.meta.url);
await access(new URL("index.json", from));
await cp(from, to, { recursive: true });
