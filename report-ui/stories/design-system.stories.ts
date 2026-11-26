import { h } from "vue";
import DataValue from "@seikika/ui/components/DataValue.vue";
import LevelBadge from "@seikika/ui/components/LevelBadge.vue";
import PageFrame from "@seikika/ui/components/PageFrame.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";

export default { title: "Design system" };

export const Primitives = {
  render: () => ({
    setup: () => () => h(PageFrame, {}, {
      default: () => [
        h(PageHeader, { title: "Design system" }),
        h(SectionCard, { title: "Levels" }, {
          default: () => h("ul", ["none", "bronze", "silver", "gold", "skipped"].map(level => h("li", [h(LevelBadge, { level })]))),
        }),
        h(SectionCard, { title: "Catalog values" }, {
          default: () => h("ul", [
            h("li", [h(DataValue, { value: ["service:checkout", true], entityHref: id => "/e/" + encodeURIComponent(id) })]),
            h("li", [h(DataValue, { value: { redacted: true, visible_to: "owners" } })]),
          ]),
        }),
      ],
    }),
  }),
};
