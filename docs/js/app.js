const currentYear = document.getElementById("current-year");
const navToggle = document.querySelector(".nav-toggle");
const siteHeader = document.querySelector(".site-header");
const siteNavLinks = document.querySelectorAll(".site-nav a");
const heroDllCount = document.getElementById("hero-dll-count");

if (currentYear) {
  currentYear.textContent = new Date().getFullYear();
}

if (navToggle && siteHeader) {
  navToggle.addEventListener("click", () => {
    const isOpen = siteHeader.classList.toggle("nav-open");
    navToggle.setAttribute("aria-expanded", String(isOpen));
  });

  siteNavLinks.forEach((link) => {
    link.addEventListener("click", () => {
      siteHeader.classList.remove("nav-open");
      navToggle.setAttribute("aria-expanded", "false");
    });
  });
}

async function hydrateHeroDllCount() {
  if (!heroDllCount) {
    return;
  }

  try {
    const response = await fetch("data/dll-support.json", { cache: "no-store" });
    if (!response.ok) {
      return;
    }

    const data = await response.json();
    if (!Array.isArray(data.dlls)) {
      return;
    }

    heroDllCount.textContent = String(data.dlls.length);
  } catch {
    // Keep default fallback value if support data cannot be fetched.
  }
}

hydrateHeroDllCount();