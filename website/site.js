const menuButton = document.querySelector('.menu-toggle');
const navigation = document.querySelector('.site-nav');

menuButton?.addEventListener('click', () => {
  const open = menuButton.getAttribute('aria-expanded') === 'true';
  menuButton.setAttribute('aria-expanded', String(!open));
  navigation?.classList.toggle('open', !open);
});

navigation?.addEventListener('click', (event) => {
  if (event.target instanceof HTMLAnchorElement) {
    navigation.classList.remove('open');
    menuButton?.setAttribute('aria-expanded', 'false');
  }
});

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape' && navigation?.classList.contains('open')) {
    navigation.classList.remove('open');
    menuButton?.setAttribute('aria-expanded', 'false');
    menuButton?.focus();
  }
});

const desktopNavigation = window.matchMedia('(min-width: 901px)');
desktopNavigation.addEventListener('change', (event) => {
  if (event.matches) {
    navigation?.classList.remove('open');
    menuButton?.setAttribute('aria-expanded', 'false');
  }
});

const codeTabs = Array.from(document.querySelectorAll('.code-tab'));
const codePanels = Array.from(document.querySelectorAll('[data-code]'));

function activateCodeTab(tab, remember = true) {
  const language = tab.getAttribute('data-lang');
  codeTabs.forEach((candidate) => {
    const active = candidate === tab;
    candidate.classList.toggle('active', active);
    candidate.setAttribute('aria-selected', String(active));
    candidate.setAttribute('tabindex', active ? '0' : '-1');
  });
  codePanels.forEach((panel) => {
    panel.hidden = panel.getAttribute('data-code') !== language;
  });

  if (remember) {
    try {
      window.localStorage.setItem('quick-start-language', language);
    } catch (_) {
      // The tabs remain fully usable when storage is disabled.
    }
  }
}

codeTabs.forEach((tab, index) => {
  const language = tab.getAttribute('data-lang');
  const panel = codePanels.find((candidate) => candidate.getAttribute('data-code') === language);
  tab.id = `quick-start-tab-${language}`;
  if (panel) {
    panel.id = `quick-start-panel-${language}`;
    panel.setAttribute('role', 'tabpanel');
    panel.setAttribute('aria-labelledby', tab.id);
    tab.setAttribute('aria-controls', panel.id);
  }

  tab.addEventListener('click', () => activateCodeTab(tab));
  tab.addEventListener('keydown', (event) => {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    let nextIndex = index;
    if (event.key === 'ArrowLeft') nextIndex = (index - 1 + codeTabs.length) % codeTabs.length;
    if (event.key === 'ArrowRight') nextIndex = (index + 1) % codeTabs.length;
    if (event.key === 'Home') nextIndex = 0;
    if (event.key === 'End') nextIndex = codeTabs.length - 1;
    activateCodeTab(codeTabs[nextIndex]);
    codeTabs[nextIndex].focus();
  });
});

let preferredLanguage = null;
try {
  preferredLanguage = window.localStorage.getItem('quick-start-language');
} catch (_) {
  // Keep the HTML default when storage is disabled.
}
const preferredTab = codeTabs.find((tab) => tab.getAttribute('data-lang') === preferredLanguage);
if (codeTabs.length > 0) {
  activateCodeTab(preferredTab || codeTabs[0], false);
}
