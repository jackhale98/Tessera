import { d as attr, g as escape_html } from "../../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../../chunks/exports.js";
import "../../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/state.svelte.js";
/* empty css                                                               */
import "../../../../../chunks/project.js";
import { M as MobileHeader } from "../../../../../chunks/MobileHeader.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let title = "";
    let deviation_type = "temporary";
    let category = "material";
    let risk_level = "low";
    let description = "";
    let author = "";
    let submitting = false;
    MobileHeader($$renderer2, { title: "New Deviation", backHref: "/more/deviations" });
    $$renderer2.push(`<!----> <div class="form-page svelte-12dssoo"><form class="svelte-12dssoo"><div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-title">Title *</label> <input id="dev-title" type="text" class="field-input svelte-12dssoo" placeholder="Deviation title"${attr("value", title)} required=""/></div> <div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-type">Deviation Type</label> `);
    $$renderer2.select(
      { id: "dev-type", class: "field-select", value: deviation_type },
      ($$renderer3) => {
        $$renderer3.option({ value: "temporary" }, ($$renderer4) => {
          $$renderer4.push(`Temporary`);
        });
        $$renderer3.option({ value: "permanent" }, ($$renderer4) => {
          $$renderer4.push(`Permanent`);
        });
        $$renderer3.option({ value: "emergency" }, ($$renderer4) => {
          $$renderer4.push(`Emergency`);
        });
      },
      "svelte-12dssoo"
    );
    $$renderer2.push(`</div> <div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-category">Category</label> `);
    $$renderer2.select(
      { id: "dev-category", class: "field-select", value: category },
      ($$renderer3) => {
        $$renderer3.option({ value: "material" }, ($$renderer4) => {
          $$renderer4.push(`Material`);
        });
        $$renderer3.option({ value: "process" }, ($$renderer4) => {
          $$renderer4.push(`Process`);
        });
        $$renderer3.option({ value: "equipment" }, ($$renderer4) => {
          $$renderer4.push(`Equipment`);
        });
        $$renderer3.option({ value: "tooling" }, ($$renderer4) => {
          $$renderer4.push(`Tooling`);
        });
        $$renderer3.option({ value: "specification" }, ($$renderer4) => {
          $$renderer4.push(`Specification`);
        });
        $$renderer3.option({ value: "documentation" }, ($$renderer4) => {
          $$renderer4.push(`Documentation`);
        });
      },
      "svelte-12dssoo"
    );
    $$renderer2.push(`</div> <div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-risk">Risk Level</label> `);
    $$renderer2.select(
      { id: "dev-risk", class: "field-select", value: risk_level },
      ($$renderer3) => {
        $$renderer3.option({ value: "low" }, ($$renderer4) => {
          $$renderer4.push(`Low`);
        });
        $$renderer3.option({ value: "medium" }, ($$renderer4) => {
          $$renderer4.push(`Medium`);
        });
        $$renderer3.option({ value: "high" }, ($$renderer4) => {
          $$renderer4.push(`High`);
        });
      },
      "svelte-12dssoo"
    );
    $$renderer2.push(`</div> <div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-desc">Description</label> <textarea id="dev-desc" class="field-textarea svelte-12dssoo" placeholder="Describe the deviation..." rows="4">`);
    const $$body = escape_html(description);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea></div> <div class="field svelte-12dssoo"><label class="field-label svelte-12dssoo" for="dev-author">Author *</label> <input id="dev-author" type="text" class="field-input svelte-12dssoo" placeholder="Your name"${attr("value", author)} required=""/></div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <button type="submit" class="submit-btn svelte-12dssoo"${attr("disabled", submitting, true)}>${escape_html("Create Deviation")}</button></form></div>`);
  });
}
export {
  _page as default
};
