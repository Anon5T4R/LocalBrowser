import { describe, expect, it } from "vitest";
import { hostOf, normalizeUrl, slugify } from "../backend";
import { PRESETS } from "../catalog";

describe("slugify", () => {
  it("minúsculo, sem acentos, separadores em traço", () => {
    expect(slugify("YouTube Music")).toBe("youtube-music");
    expect(slugify("Café com Açúcar")).toBe("cafe-com-acucar");
    expect(slugify("  --Spotify--  ")).toBe("spotify");
  });
  it("fallback quando não sobra nada", () => {
    expect(slugify("!!!")).toBe("app");
    expect(slugify("")).toBe("app");
  });
});

describe("normalizeUrl", () => {
  it("adiciona https quando falta esquema", () => {
    expect(normalizeUrl("open.spotify.com")).toBe("https://open.spotify.com");
    expect(normalizeUrl(" open.spotify.com ")).toBe("https://open.spotify.com");
  });
  it("não mexe quando já tem esquema", () => {
    expect(normalizeUrl("http://a.b")).toBe("http://a.b");
    expect(normalizeUrl("https://a.b/x")).toBe("https://a.b/x");
  });
  it("vazio fica vazio", () => {
    expect(normalizeUrl("")).toBe("");
  });
});

describe("hostOf", () => {
  it("extrai o host e tira www", () => {
    expect(hostOf("https://www.youtube.com/watch")).toBe("youtube.com");
    expect(hostOf("open.spotify.com")).toBe("open.spotify.com");
  });
  it("URL inválida vaza vazia", () => {
    expect(hostOf("http://")).toBe("");
  });
});

describe("PRESETS", () => {
  it("ids únicos e URLs absolutas válidas", () => {
    const ids = new Set(PRESETS.map((p) => p.id));
    expect(ids.size).toBe(PRESETS.length);
    for (const p of PRESETS) {
      expect(new URL(p.url).protocol).toBe("https:");
      expect(p.emoji.length).toBeGreaterThan(0);
    }
  });
});
