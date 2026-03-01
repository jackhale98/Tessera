import { d as attr, n as stringify, g as escape_html, h as derived, c as store_get, u as unsubscribe_stores } from "../../../../chunks/root.js";
import { p as page } from "../../../../chunks/stores.js";
/* empty css                                                            */
import "../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../chunks/MobileHeader.js";
import "../../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import "clsx";
import { S as Search } from "../../../../chunks/search.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    const TYPE_LABELS = {
      req: "Requirements",
      risk: "Risks",
      test: "Tests",
      rslt: "Results",
      cmp: "Components",
      asm: "Assemblies",
      proc: "Processes",
      ctrl: "Controls",
      work: "Work Instructions",
      lot: "Lots",
      dev: "Deviations",
      ncr: "NCRs",
      capa: "CAPAs"
    };
    let search = "";
    let typeParam = derived(() => store_get($$store_subs ??= {}, "$page", page).params.type);
    let typeLabel = derived(() => TYPE_LABELS[typeParam()] ?? typeParam().toUpperCase());
    MobileHeader($$renderer2, { title: typeLabel(), backHref: "/browse" });
    $$renderer2.push(`<!----> <div class="page svelte-1ho7vpg"><div class="search-bar svelte-1ho7vpg">`);
    Search($$renderer2, { size: 16, class: "search-icon" });
    $$renderer2.push(`<!----> <input type="text"${attr("placeholder", `Search ${stringify(typeLabel().toLowerCase())}...`)}${attr("value", search)} class="search-input svelte-1ho7vpg"/></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-state svelte-1ho7vpg"><div class="loading-spinner svelte-1ho7vpg"></div> <p>Loading ${escape_html(typeLabel().toLowerCase())}...</p></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
