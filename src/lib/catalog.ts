/** Sugestões de "web apps" — nomes/emojis são marcas, não se traduzem. */

export interface Preset {
  id: string;
  name: string;
  url: string;
  emoji: string;
}

export const PRESETS: Preset[] = [
  { id: "spotify", name: "Spotify", url: "https://open.spotify.com", emoji: "🎧" },
  { id: "youtube-music", name: "YouTube Music", url: "https://music.youtube.com", emoji: "🎵" },
  { id: "youtube", name: "YouTube", url: "https://youtube.com", emoji: "▶️" },
  { id: "netflix", name: "Netflix", url: "https://netflix.com", emoji: "🎬" },
  { id: "prime-video", name: "Prime Video", url: "https://primevideo.com", emoji: "📺" },
  { id: "twitch", name: "Twitch", url: "https://twitch.tv", emoji: "🎮" },
  { id: "discord", name: "Discord", url: "https://discord.com/app", emoji: "💬" },
  { id: "whatsapp", name: "WhatsApp", url: "https://web.whatsapp.com", emoji: "🟢" },
  { id: "telegram", name: "Telegram", url: "https://web.telegram.org", emoji: "✈️" },
  { id: "instagram", name: "Instagram", url: "https://instagram.com", emoji: "📷" },
  { id: "reddit", name: "Reddit", url: "https://reddit.com", emoji: "👽" },
  { id: "gmail", name: "Gmail", url: "https://mail.google.com", emoji: "✉️" },
  { id: "google-maps", name: "Google Maps", url: "https://google.com/maps", emoji: "🗺️" },
  { id: "figma", name: "Figma", url: "https://figma.com", emoji: "🎨" },
  { id: "notion", name: "Notion", url: "https://notion.so", emoji: "📝" },
  { id: "chatgpt", name: "ChatGPT", url: "https://chatgpt.com", emoji: "🤖" },
  { id: "github", name: "GitHub", url: "https://github.com", emoji: "🐙" },
  { id: "wikipedia", name: "Wikipedia", url: "https://wikipedia.org", emoji: "📚" },
  { id: "duolingo", name: "Duolingo", url: "https://duolingo.com", emoji: "🦉" },
  { id: "deezer", name: "Deezer", url: "https://deezer.com", emoji: "🎼" },
];
