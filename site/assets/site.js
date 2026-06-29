// Talos product-site script.
//
// Plain, dependency-free JavaScript. Two responsibilities only:
//   1. Stamp the current year in the footer.
//   2. Wire up a small "Copy" affordance for fenced code blocks.
//
// No analytics, no third-party scripts, no network calls.

(function () {
  "use strict";

  function stampYear() {
    var el = document.querySelector("[data-site-year]");
    if (el) {
      el.textContent = String(new Date().getFullYear());
    }
  }

  function wireCopyButtons() {
    var blocks = document.querySelectorAll(".talos-codeblock");
    blocks.forEach(function (block) {
      var pre = block.querySelector("pre");
      if (!pre) return;
      var btn = document.createElement("button");
      btn.type = "button";
      btn.className = "talos-codeblock__copy";
      btn.textContent = "Copy";
      btn.setAttribute("aria-label", "Copy code to clipboard");
      btn.addEventListener("click", function () {
        var text = pre.innerText;
        var done = function () {
          btn.textContent = "Copied";
          setTimeout(function () {
            btn.textContent = "Copy";
          }, 1500);
        };
        if (
          navigator.clipboard &&
          typeof navigator.clipboard.writeText === "function"
        ) {
          navigator.clipboard.writeText(text).then(done, done);
        } else {
          // Fallback for very old browsers
          var ta = document.createElement("textarea");
          ta.value = text;
          ta.setAttribute("readonly", "");
          ta.style.position = "absolute";
          ta.style.left = "-9999px";
          document.body.appendChild(ta);
          ta.select();
          try {
            document.execCommand("copy");
            done();
          } catch (e) {
            // Silent failure: the user can still select and copy manually.
          }
          document.body.removeChild(ta);
        }
      });
      block.appendChild(btn);
    });
  }

  function markCurrentNav() {
    // Mark the nav link that matches the current page. Pages set
    // <body data-page="install"> etc.; fallback to path matching.
    var path = (window.location.pathname || "").toLowerCase();
    var explicit = (
      document.body && document.body.getAttribute("data-page")
    ) || "";
    var slug = explicit;
    if (!slug) {
      if (path === "/" || path.endsWith("/index.html")) slug = "home";
      else if (path.endsWith("/install.html")) slug = "install";
      else if (path.endsWith("/capabilities.html")) slug = "capabilities";
      else if (path.endsWith("/safety.html")) slug = "safety";
      else if (path.endsWith("/roadmap.html")) slug = "roadmap";
      else if (path.endsWith("/releases.html")) slug = "releases";
      else slug = "";
    }
    if (!slug) return;
    var link = document.querySelector(
      '.talos-nav a[data-nav-slug="' + slug + '"]'
    );
    if (link) link.setAttribute("aria-current", "page");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      stampYear();
      wireCopyButtons();
      markCurrentNav();
    });
  } else {
    stampYear();
    wireCopyButtons();
    markCurrentNav();
  }
})();
