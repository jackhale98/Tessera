import { d as attr, e as ensure_array_like, f as attr_class, g as escape_html } from "../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../chunks/exports.js";
import "../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/state.svelte.js";
/* empty css                                                         */
import "../../../chunks/project.js";
import { M as MobileHeader } from "../../../chunks/MobileHeader.js";
import "../../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import "clsx";
import { S as Search } from "../../../chunks/search.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let search = "";
    let activeFilter = "all";
    const filters = [
      { id: "all", label: "All" },
      { id: "in_progress", label: "In Progress" },
      { id: "on_hold", label: "On Hold" },
      { id: "completed", label: "Completed" }
    ];
    MobileHeader($$renderer2, { title: "Lots" });
    $$renderer2.push(`<!----> <div class="page svelte-1h6m9g1"><div class="search-bar svelte-1h6m9g1">`);
    Search($$renderer2, { size: 16, class: "search-icon" });
    $$renderer2.push(`<!----> <input type="text" placeholder="Search lots..."${attr("value", search)} class="search-input svelte-1h6m9g1"/></div> <div class="filter-chips no-scrollbar svelte-1h6m9g1"><!--[-->`);
    const each_array = ensure_array_like(filters);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let filter = each_array[$$index];
      $$renderer2.push(`<button${attr_class("chip svelte-1h6m9g1", void 0, { "active": activeFilter === filter.id })}>${escape_html(filter.label)}</button>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading-state svelte-1h6m9g1"><div class="loading-spinner svelte-1h6m9g1"></div> <p>Loading lots...</p></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
