export type Theme = "light" | "dark";
export type Accent = "ember" | "teal" | "blue" | "violet" | "rose";
export type Density = "default" | "compact";

export const ThemeManager = {
  getTheme(): Theme {
    return (document.documentElement.getAttribute("data-theme") as Theme) || "light";
  },

  setTheme(theme: Theme) {
    document.documentElement.setAttribute("data-theme", theme);
  },

  getAccent(): Accent {
    return (document.documentElement.getAttribute("data-accent") as Accent) || "ember";
  },

  setAccent(accent: Accent) {
    document.documentElement.setAttribute("data-accent", accent);
  },

  getDensity(): Density {
    return (document.documentElement.getAttribute("data-density") as Density) || "default";
  },

  setDensity(density: Density) {
    document.documentElement.setAttribute("data-density", density);
  },

  init(theme: Theme = "light", accent: Accent = "ember", density: Density = "default") {
    this.setTheme(theme);
    this.setAccent(accent);
    this.setDensity(density);
  },
};
