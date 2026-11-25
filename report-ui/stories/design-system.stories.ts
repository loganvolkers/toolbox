import { h } from "vue";
import DataValue from "@toolbox-dev/ui/components/DataValue.vue";
import LevelBadge from "@toolbox-dev/ui/components/LevelBadge.vue";
import PageFrame from "@toolbox-dev/ui/components/PageFrame.vue";
import PageHeader from "@toolbox-dev/ui/components/PageHeader.vue";
import SectionCard from "@toolbox-dev/ui/components/SectionCard.vue";

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
