import { s as sanitize_props, o as rest_props, p as fallback, k as attributes, l as clsx, e as ensure_array_like, q as element, b as slot, t as bind_props } from "./root.js";
import { i as derived, w as writable } from "./exports.js";
const defaultAttributes = {
  xmlns: "http://www.w3.org/2000/svg",
  width: 24,
  height: 24,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  "stroke-width": 2,
  "stroke-linecap": "round",
  "stroke-linejoin": "round"
};
function Icon($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const $$restProps = rest_props($$sanitized_props, [
    "name",
    "color",
    "size",
    "strokeWidth",
    "absoluteStrokeWidth",
    "iconNode"
  ]);
  $$renderer.component(($$renderer2) => {
    let name = fallback($$props["name"], void 0);
    let color = fallback($$props["color"], "currentColor");
    let size = fallback($$props["size"], 24);
    let strokeWidth = fallback($$props["strokeWidth"], 2);
    let absoluteStrokeWidth = fallback($$props["absoluteStrokeWidth"], false);
    let iconNode = fallback($$props["iconNode"], () => [], true);
    const mergeClasses = (...classes) => classes.filter((className, index, array) => {
      return Boolean(className) && array.indexOf(className) === index;
    }).join(" ");
    $$renderer2.push(`<svg${attributes(
      {
        ...defaultAttributes,
        ...$$restProps,
        width: size,
        height: size,
        stroke: color,
        "stroke-width": absoluteStrokeWidth ? Number(strokeWidth) * 24 / Number(size) : strokeWidth,
        class: clsx(mergeClasses("lucide-icon", "lucide", name ? `lucide-${name}` : "", $$sanitized_props.class))
      },
      void 0,
      void 0,
      void 0,
      3
    )}><!--[-->`);
    const each_array = ensure_array_like(iconNode);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let [tag, attrs] = each_array[$$index];
      element($$renderer2, tag, () => {
        $$renderer2.push(`${attributes({ ...attrs }, void 0, void 0, void 0, 3)}`);
      });
    }
    $$renderer2.push(`<!--]--><!--[-->`);
    slot($$renderer2, $$props, "default", {});
    $$renderer2.push(`<!--]--></svg>`);
    bind_props($$props, {
      name,
      color,
      size,
      strokeWidth,
      absoluteStrokeWidth,
      iconNode
    });
  });
}
const projectInfo = writable(null);
const isProjectOpen = derived(projectInfo, ($info) => $info !== null);
const projectName = derived(projectInfo, ($info) => $info?.name ?? "");
derived(projectInfo, ($info) => $info?.path ?? "");
derived(projectInfo, ($info) => $info?.author ?? "");
const entityCounts = derived(projectInfo, ($info) => $info?.entity_counts ?? null);
const totalEntities = derived(entityCounts, ($counts) => {
  if (!$counts) return 0;
  return $counts.requirements + $counts.risks + ($counts.hazards ?? 0) + $counts.tests + $counts.results + $counts.components + $counts.assemblies + $counts.features + $counts.mates + $counts.stackups + $counts.processes + $counts.controls + $counts.work_instructions + $counts.lots + $counts.deviations + $counts.ncrs + $counts.capas + $counts.quotes + $counts.suppliers + ($counts.actions ?? 0);
});
export {
  Icon as I,
  entityCounts as e,
  isProjectOpen as i,
  projectName as p,
  totalEntities as t
};
