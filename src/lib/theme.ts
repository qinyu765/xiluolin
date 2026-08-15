import { setTheme as setApplicationTheme } from "@tauri-apps/api/app";

type SetTheme = (theme: "light") => Promise<void>;

export async function enforceLightTheme(
  setTheme: SetTheme = setApplicationTheme,
) {
  document.documentElement.classList.remove("dark");
  document.documentElement.style.colorScheme = "light";
  await setTheme("light");
}
