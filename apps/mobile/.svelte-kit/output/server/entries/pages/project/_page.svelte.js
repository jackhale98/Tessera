import { s as sanitize_props, a as spread_props, b as slot, d as attr } from "../../../chunks/root.js";
import { I as Icon } from "../../../chunks/project.js";
import "@sveltejs/kit/internal";
import "../../../chunks/exports.js";
import "../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/state.svelte.js";
import { C as Chevron_right } from "../../../chunks/chevron-right.js";
function Folder_open($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const iconNode = [
    [
      "path",
      {
        "d": "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "folder-open" },
    $$sanitized_props,
    {
      /**
       * @component @name FolderOpen
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJtNiAxNCAxLjUtMi45QTIgMiAwIDAgMSA5LjI0IDEwSDIwYTIgMiAwIDAgMSAxLjk0IDIuNWwtMS41NCA2YTIgMiAwIDAgMS0xLjk1IDEuNUg0YTIgMiAwIDAgMS0yLTJWNWEyIDIgMCAwIDEgMi0yaDMuOWEyIDIgMCAwIDEgMS42OS45bC44MSAxLjJhMiAyIDAgMCAwIDEuNjcuOUgxOGEyIDIgMCAwIDEgMiAydjIiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/folder-open
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let loading = false;
    $$renderer2.push(`<div class="project-screen svelte-urn6fi"><div class="hero svelte-urn6fi"><div class="logo-container svelte-urn6fi"><div class="logo svelte-urn6fi"><span class="logo-letter svelte-urn6fi">T</span></div></div> <h1 class="hero-title svelte-urn6fi">Tessera</h1> <p class="hero-subtitle svelte-urn6fi">Engineering Artifact Management</p></div> <div class="actions svelte-urn6fi"><button class="open-btn svelte-urn6fi"${attr("disabled", loading, true)}><div class="btn-icon svelte-urn6fi">`);
    Folder_open($$renderer2, { size: 24 });
    $$renderer2.push(`<!----></div> <div class="btn-text svelte-urn6fi"><span class="btn-label svelte-urn6fi">Open Project</span> <span class="btn-desc svelte-urn6fi">Select a TDT project folder</span></div> `);
    Chevron_right($$renderer2, { size: 20, class: "btn-chevron" });
    $$renderer2.push(`<!----></button> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div> <p class="footer-note svelte-urn6fi">Projects are stored locally on your device. Sync via desktop app.</p></div>`);
  });
}
export {
  _page as default
};
