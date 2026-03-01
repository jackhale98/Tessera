import { g as escape_html, d as attr, f as attr_class, h as derived, c as store_get, u as unsubscribe_stores } from "../../../../../chunks/root.js";
import { p as page } from "../../../../../chunks/stores.js";
import "@sveltejs/kit/internal";
import "../../../../../chunks/exports.js";
import "../../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/state.svelte.js";
/* empty css                                                               */
import "../../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../../chunks/MobileHeader.js";
import { T as Triangle_alert } from "../../../../../chunks/triangle-alert.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let title = "";
    let severity = "major";
    let ncr_type = "internal";
    let description = "";
    let lotId = derived(() => store_get($$store_subs ??= {}, "$page", page).url.searchParams.get("lotId"));
    let canSubmit = derived(() => title.trim().length > 0 && true);
    MobileHeader($$renderer2, { title: "New NCR", backHref: "/quality/ncrs" });
    $$renderer2.push(`<!----> <div class="form-page svelte-1v155qc">`);
    if (lotId()) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="lot-banner svelte-1v155qc">`);
      Triangle_alert($$renderer2, { size: 16 });
      $$renderer2.push(`<!----> <span>Creating NCR for lot <strong>${escape_html(lotId())}</strong></span></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <form class="svelte-1v155qc"><div class="form-group svelte-1v155qc"><label class="form-label svelte-1v155qc" for="ncr-title">Title</label> <input id="ncr-title" type="text" class="form-input svelte-1v155qc" placeholder="Describe the non-conformance..."${attr("value", title)} required=""/></div> <div class="form-group svelte-1v155qc"><label class="form-label svelte-1v155qc">Severity</label> <div class="toggle-group svelte-1v155qc"><button type="button"${attr_class("toggle-btn svelte-1v155qc", void 0, {
      "active": severity === "minor",
      "minor": severity === "minor"
    })}>Minor</button> <button type="button"${attr_class("toggle-btn svelte-1v155qc", void 0, {
      "active": severity === "major",
      "major": severity === "major"
    })}>Major</button> <button type="button"${attr_class("toggle-btn svelte-1v155qc", void 0, {
      "active": severity === "critical",
      "critical": severity === "critical"
    })}>Critical</button></div></div> <div class="form-group svelte-1v155qc"><label class="form-label svelte-1v155qc" for="ncr-type">Type</label> `);
    $$renderer2.select(
      { id: "ncr-type", class: "form-select", value: ncr_type },
      ($$renderer3) => {
        $$renderer3.option({ value: "internal" }, ($$renderer4) => {
          $$renderer4.push(`Internal`);
        });
        $$renderer3.option({ value: "supplier" }, ($$renderer4) => {
          $$renderer4.push(`Supplier`);
        });
        $$renderer3.option({ value: "customer" }, ($$renderer4) => {
          $$renderer4.push(`Customer`);
        });
      },
      "svelte-1v155qc"
    );
    $$renderer2.push(`</div> <div class="form-group svelte-1v155qc"><label class="form-label svelte-1v155qc" for="ncr-desc">Description</label> <textarea id="ncr-desc" class="form-textarea svelte-1v155qc" placeholder="Provide details about the non-conformance..."${attr("rows", 4)}>`);
    const $$body = escape_html(description);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea></div> <button type="submit" class="submit-btn svelte-1v155qc"${attr("disabled", !canSubmit(), true)}>`);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`Create NCR`);
    }
    $$renderer2.push(`<!--]--></button></form></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
