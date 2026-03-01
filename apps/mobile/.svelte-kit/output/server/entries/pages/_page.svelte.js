import { s as sanitize_props, a as spread_props, b as slot, k as attributes, l as clsx$1, g as escape_html, h as derived, d as attr, m as attr_style, n as stringify, c as store_get, e as ensure_array_like, u as unsubscribe_stores } from "../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../chunks/state.svelte.js";
import { I as Icon, p as projectName, t as totalEntities, e as entityCounts } from "../../chunks/project.js";
/* empty css                                                      */
import { M as MobileHeader } from "../../chunks/MobileHeader.js";
import { b as badgeVariants } from "../../chunks/EntityCard.svelte_svelte_type_style_lang.js";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { t as truncateEntityId, g as getEntityColorSolid, S as Shield_alert } from "../../chunks/entities.js";
import { C as Chevron_right } from "../../chunks/chevron-right.js";
import { P as Package } from "../../chunks/package.js";
import { T as Triangle_alert } from "../../chunks/triangle-alert.js";
function Activity($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const iconNode = [
    [
      "path",
      {
        "d": "M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "activity" },
    $$sanitized_props,
    {
      /**
       * @component @name Activity
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMjIgMTJoLTIuNDhhMiAyIDAgMCAwLTEuOTMgMS40NmwtMi4zNSA4LjM2YS4yNS4yNSAwIDAgMS0uNDggMEw5LjI0IDIuMThhLjI1LjI1IDAgMCAwLS40OCAwbC0yLjM1IDguMzZBMiAyIDAgMCAxIDQuNDkgMTJIMiIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/activity
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
function Refresh_cw($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  const iconNode = [
    [
      "path",
      { "d": "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
    ],
    ["path", { "d": "M21 3v5h-5" }],
    [
      "path",
      { "d": "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
    ],
    ["path", { "d": "M8 16H3v5" }]
  ];
  Icon($$renderer, spread_props([
    { name: "refresh-cw" },
    $$sanitized_props,
    {
      /**
       * @component @name RefreshCw
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMyAxMmE5IDkgMCAwIDEgOS05IDkuNzUgOS43NSAwIDAgMSA2Ljc0IDIuNzRMMjEgOCIgLz4KICA8cGF0aCBkPSJNMjEgM3Y1aC01IiAvPgogIDxwYXRoIGQ9Ik0yMSAxMmE5IDkgMCAwIDEtOSA5IDkuNzUgOS43NSAwIDAgMS02Ljc0LTIuNzRMMyAxNiIgLz4KICA8cGF0aCBkPSJNOCAxNkgzdjUiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/refresh-cw
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
function cn(...inputs) {
  return twMerge(clsx(inputs));
}
function Badge($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let {
      class: className,
      variant = "default",
      children,
      $$slots,
      $$events,
      ...restProps
    } = $$props;
    $$renderer2.push(`<div${attributes({
      class: clsx$1(cn(badgeVariants({ variant }), className)),
      ...restProps
    })}>`);
    children?.($$renderer2);
    $$renderer2.push(`<!----></div>`);
  });
}
function StatusBadge($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { status, class: className } = $$props;
    const statusConfig = {
      draft: { variant: "outline", label: "Draft" },
      review: { variant: "secondary", label: "Review" },
      approved: { variant: "default", label: "Approved" },
      released: { variant: "default", label: "Released" },
      obsolete: { variant: "destructive", label: "Obsolete" }
    };
    const config = derived(() => statusConfig[status.toLowerCase()] ?? { variant: "outline", label: status });
    Badge($$renderer2, {
      variant: config().variant,
      class: cn("capitalize", className),
      children: ($$renderer3) => {
        $$renderer3.push(`<!---->${escape_html(config().label)}`);
      },
      $$slots: { default: true }
    });
  });
}
function EntityCard($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { id, title, subtitle, status, prefix, href, meta, onclick } = $$props;
    let entityPrefix = derived(() => prefix || id.split("-")[0]);
    let accentColor = derived(() => getEntityColorSolid(entityPrefix()));
    if (href) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<a${attr("href", href)} class="entity-card touch-highlight svelte-860i8z"${attr("data-accent", entityPrefix())}><div class="card-accent svelte-860i8z"${attr_style(`background-color: ${stringify(accentColor())}`)}></div> <div class="card-body svelte-860i8z"><div class="card-top svelte-860i8z"><span class="card-id svelte-860i8z">${escape_html(truncateEntityId(id))}</span> `);
      if (status) {
        $$renderer2.push("<!--[-->");
        StatusBadge($$renderer2, { status });
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <h3 class="card-title svelte-860i8z">${escape_html(title)}</h3> `);
      if (subtitle) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<p class="card-subtitle svelte-860i8z">${escape_html(subtitle)}</p>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      if (meta) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<p class="card-meta svelte-860i8z">${escape_html(meta)}</p>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></a>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<button class="entity-card touch-highlight svelte-860i8z" type="button"><div class="card-accent svelte-860i8z"${attr_style(`background-color: ${stringify(accentColor())}`)}></div> <div class="card-body svelte-860i8z"><div class="card-top svelte-860i8z"><span class="card-id svelte-860i8z">${escape_html(truncateEntityId(id))}</span> `);
      if (status) {
        $$renderer2.push("<!--[-->");
        StatusBadge($$renderer2, { status });
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <h3 class="card-title svelte-860i8z">${escape_html(title)}</h3> `);
      if (subtitle) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<p class="card-subtitle svelte-860i8z">${escape_html(subtitle)}</p>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      if (meta) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<p class="card-meta svelte-860i8z">${escape_html(meta)}</p>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></button>`);
    }
    $$renderer2.push(`<!--]-->`);
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let activeLots = [];
    let openNcrs = [];
    let loading = true;
    const quickStats = derived(() => [
      {
        label: "Total Entities",
        value: store_get($$store_subs ??= {}, "$totalEntities", totalEntities),
        icon: Activity,
        color: "var(--theme-primary)"
      },
      {
        label: "Active Lots",
        value: activeLots.length,
        icon: Package,
        color: "var(--theme-success)"
      },
      {
        label: "Open NCRs",
        value: openNcrs.length,
        icon: Shield_alert,
        color: "var(--theme-error)"
      },
      {
        label: "Entity Types",
        value: Object.keys(store_get($$store_subs ??= {}, "$entityCounts", entityCounts)).filter((k) => store_get($$store_subs ??= {}, "$entityCounts", entityCounts)[k] > 0).length,
        icon: Triangle_alert,
        color: "var(--theme-warning)"
      }
    ]);
    MobileHeader($$renderer2, {
      title: store_get($$store_subs ??= {}, "$projectName", projectName) || "Tessera",
      children: ($$renderer3) => {
        $$renderer3.push(`<button class="refresh-btn svelte-1uha8ag"${attr("disabled", loading, true)} aria-label="Refresh">`);
        Refresh_cw($$renderer3, { size: 18, class: "spin" });
        $$renderer3.push(`<!----></button>`);
      }
    });
    $$renderer2.push(`<!----> <div class="dashboard svelte-1uha8ag"><div class="stats-grid svelte-1uha8ag"><!--[-->`);
    const each_array = ensure_array_like(quickStats());
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let stat = each_array[$$index];
      $$renderer2.push(`<div class="stat-card svelte-1uha8ag"><div class="stat-icon svelte-1uha8ag"${attr_style(`background-color: color-mix(in oklch, ${stringify(stat.color)} 15%, transparent); color: ${stringify(stat.color)}`)}>`);
      if (stat.icon) {
        $$renderer2.push("<!--[-->");
        stat.icon($$renderer2, { size: 18 });
        $$renderer2.push("<!--]-->");
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push("<!--]-->");
      }
      $$renderer2.push(`</div> <span class="stat-value svelte-1uha8ag">${escape_html(stat.value)}</span> <span class="stat-label svelte-1uha8ag">${escape_html(stat.label)}</span></div>`);
    }
    $$renderer2.push(`<!--]--></div> <section class="section svelte-1uha8ag"><div class="section-header svelte-1uha8ag"><h2 class="section-title svelte-1uha8ag">Active Lots</h2> <a href="/lots" class="section-link svelte-1uha8ag">See all `);
    Chevron_right($$renderer2, { size: 14 });
    $$renderer2.push(`<!----></a></div> `);
    if (activeLots.length === 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="empty-state svelte-1uha8ag">`);
      Package($$renderer2, { size: 32, strokeWidth: 1.2 });
      $$renderer2.push(`<!----> <p class="svelte-1uha8ag">No active lots</p></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div class="card-list svelte-1uha8ag"><!--[-->`);
      const each_array_1 = ensure_array_like(activeLots);
      for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
        let lot = each_array_1[$$index_1];
        EntityCard($$renderer2, {
          id: lot.id,
          title: lot.title,
          subtitle: lot.lot_number ? `Lot #${lot.lot_number}` : void 0,
          status: lot.lot_status,
          prefix: "LOT",
          href: `/lots/${stringify(lot.id)}`,
          meta: lot.quantity ? `Qty: ${lot.quantity}` : void 0
        });
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></section> <section class="section svelte-1uha8ag"><div class="section-header svelte-1uha8ag"><h2 class="section-title svelte-1uha8ag">Open NCRs</h2> <a href="/quality/ncrs" class="section-link svelte-1uha8ag">See all `);
    Chevron_right($$renderer2, { size: 14 });
    $$renderer2.push(`<!----></a></div> `);
    if (openNcrs.length === 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="empty-state svelte-1uha8ag">`);
      Shield_alert($$renderer2, { size: 32, strokeWidth: 1.2 });
      $$renderer2.push(`<!----> <p class="svelte-1uha8ag">No open NCRs</p></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div class="card-list svelte-1uha8ag"><!--[-->`);
      const each_array_2 = ensure_array_like(openNcrs);
      for (let $$index_2 = 0, $$length = each_array_2.length; $$index_2 < $$length; $$index_2++) {
        let ncr = each_array_2[$$index_2];
        EntityCard($$renderer2, {
          id: ncr.id,
          title: ncr.title,
          subtitle: ncr.ncr_number || ncr.ncr_type,
          status: ncr.ncr_status,
          prefix: "NCR",
          href: `/quality/ncrs/${stringify(ncr.id)}`,
          meta: `Severity: ${ncr.severity}`
        });
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></section></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
