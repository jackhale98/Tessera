import { d as attr, e as ensure_array_like, f as attr_class, g as escape_html } from "../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/state.svelte.js";
/* empty css                                                            */
import "../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../chunks/MobileHeader.js";
import "../../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import "clsx";
import { S as Search } from "../../../../chunks/search.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let search = "";
    let activeFilter = "all";
    const filters = [
      { id: "all", label: "All" },
      { id: "open", label: "Open" },
      { id: "overdue", label: "Overdue" }
    ];
    MobileHeader($$renderer2, { title: "CAPAs", backHref: "/quality" });
    $$renderer2.push(`<!----> <div class="page svelte-1ejd8d"><div class="search-bar svelte-1ejd8d">`);
    Search($$renderer2, { size: 16, class: "search-icon" });
    $$renderer2.push(`<!----> <input type="text" placeholder="Search CAPAs..."${attr("value", search)} class="search-input svelte-1ejd8d"/></div> <div class="filter-chips no-scrollbar svelte-1ejd8d"><!--[-->`);
    const each_array = ensure_array_like(filters);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let filter = each_array[$$index];
      $$renderer2.push(`<button${attr_class("chip svelte-1ejd8d", void 0, { "active": activeFilter === filter.id })}>${escape_html(filter.label)}</button>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-state svelte-1ejd8d"><div class="loading-spinner svelte-1ejd8d"></div> <p>Loading CAPAs...</p></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
