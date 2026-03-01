import { s as sanitize_props, a as spread_props, b as slot, e as ensure_array_like, c as store_get, d as attr, f as attr_class, g as escape_html, u as unsubscribe_stores, h as derived, i as head } from "../../chunks/root.js";
import { p as page } from "../../chunks/stores.js";
/* empty css                                                      */
import { I as Icon, i as isProjectOpen } from "../../chunks/project.js";
import { P as Package } from "../../chunks/package.js";
import { S as Shield_check } from "../../chunks/shield-check.js";
import { S as Search } from "../../chunks/search.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "clsx";
import "@sveltejs/kit/internal/server";
import "../../chunks/state.svelte.js";
function House($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const iconNode = [
    [
      "path",
      { "d": "M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8" }
    ],
    [
      "path",
      {
        "d": "M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "house" },
    $$sanitized_props,
    {
      /**
       * @component @name House
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTUgMjF2LThhMSAxIDAgMCAwLTEtMWgtNGExIDEgMCAwIDAtMSAxdjgiIC8+CiAgPHBhdGggZD0iTTMgMTBhMiAyIDAgMCAxIC43MDktMS41MjhsNy02YTIgMiAwIDAgMSAyLjU4MiAwbDcgNkEyIDIgMCAwIDEgMjEgMTB2OWEyIDIgMCAwIDEtMiAySDVhMiAyIDAgMCAxLTItMnoiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/house
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
function Menu($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const iconNode = [
    ["path", { "d": "M4 5h16" }],
    ["path", { "d": "M4 12h16" }],
    ["path", { "d": "M4 19h16" }]
  ];
  Icon($$renderer, spread_props([
    { name: "menu" },
    $$sanitized_props,
    {
      /**
       * @component @name Menu
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNNCA1aDE2IiAvPgogIDxwYXRoIGQ9Ik00IDEyaDE2IiAvPgogIDxwYXRoIGQ9Ik00IDE5aDE2IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/menu
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
function BottomNav($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    const tabs = [
      { href: "/", label: "Home", icon: House, match: /^\/$/ },
      {
        href: "/lots",
        label: "Lots",
        icon: Package,
        match: /^\/lots/
      },
      {
        href: "/quality",
        label: "Quality",
        icon: Shield_check,
        match: /^\/quality/
      },
      {
        href: "/browse",
        label: "Browse",
        icon: Search,
        match: /^\/browse/
      },
      { href: "/more", label: "More", icon: Menu, match: /^\/more/ }
    ];
    function isActive(match, pathname) {
      return match.test(pathname);
    }
    $$renderer2.push(`<nav class="bottom-nav touch-none-select touch-highlight svelte-qzbt73"><div class="nav-inner svelte-qzbt73"><!--[-->`);
    const each_array = ensure_array_like(tabs);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let tab = each_array[$$index];
      const active = isActive(tab.match, store_get($$store_subs ??= {}, "$page", page).url.pathname);
      $$renderer2.push(`<a${attr("href", tab.href)}${attr_class("nav-item svelte-qzbt73", void 0, { "active": active })}${attr("aria-current", active ? "page" : void 0)}><div${attr_class("icon-container svelte-qzbt73", void 0, { "active": active })}>`);
      tab.icon($$renderer2, { size: 22, strokeWidth: active ? 2.5 : 1.8 });
      $$renderer2.push(`<!----></div> <span${attr_class("nav-label svelte-qzbt73", void 0, { "active": active })}>${escape_html(tab.label)}</span></a>`);
    }
    $$renderer2.push(`<!--]--></div></nav>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function MobileLayout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let { children } = $$props;
    let showNav = derived(() => store_get($$store_subs ??= {}, "$isProjectOpen", isProjectOpen) || store_get($$store_subs ??= {}, "$page", page).url.pathname !== "/project");
    $$renderer2.push(`<div class="mobile-layout svelte-16qo2rj"><div class="safe-top bg-background"></div> <main class="mobile-content svelte-16qo2rj">`);
    children($$renderer2);
    $$renderer2.push(`<!----></main> `);
    if (showNav()) {
      $$renderer2.push("<!--[-->");
      BottomNav($$renderer2);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <div class="safe-bottom bg-sidebar"></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    head("12qhfyh", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Tessera</title>`);
      });
      $$renderer3.push(`<meta name="description" content="Tessera - Engineering Artifact Management"/>`);
    });
    MobileLayout($$renderer2, {
      children: ($$renderer3) => {
        children($$renderer3);
        $$renderer3.push(`<!---->`);
      }
    });
  });
}
export {
  _layout as default
};
