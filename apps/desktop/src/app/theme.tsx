import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

import { api } from "../api";
import type { ThemePreference } from "../generated";

const settingsQueryKey = ["core", "settings"] as const;

interface ThemeContextValue {
  preference: ThemePreference;
  hasWarning: boolean;
  isSaving: boolean;
  setPreference(preference: ThemePreference): Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

function systemTheme() {
  return typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const settings = useQuery({
    queryKey: settingsQueryKey,
    queryFn: () => api.core.getSettings(),
    staleTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
  });
  const [system, setSystem] = useState(systemTheme);
  const [isSaving, setIsSaving] = useState(false);
  const preference = settings.data?.theme ?? "system";
  const effective = preference === "system" ? system : preference;

  useEffect(() => {
    if (typeof window.matchMedia !== "function") return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => {
      setSystem(media.matches ? "dark" : "light");
    };
    media.addEventListener("change", handleChange);
    return () => {
      media.removeEventListener("change", handleChange);
    };
  }, []);

  useEffect(() => {
    document.documentElement.dataset["theme"] = effective;
    document.documentElement.dataset["themePreference"] = preference;
    document.documentElement.style.colorScheme = effective;
  }, [effective, preference]);

  const setPreference = useCallback(
    async (next: ThemePreference) => {
      setIsSaving(true);
      try {
        const updated = await api.core.setTheme({ theme: next });
        queryClient.setQueryData(settingsQueryKey, updated);
      } finally {
        setIsSaving(false);
      }
    },
    [queryClient],
  );

  const value = useMemo(
    () => ({
      preference,
      hasWarning: settings.data?.theme_warning ?? false,
      isSaving,
      setPreference,
    }),
    [isSaving, preference, setPreference, settings.data?.theme_warning],
  );
  return (
    <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useTheme() {
  const value = useContext(ThemeContext);
  if (!value) {
    throw new Error("useTheme must be used inside ThemeProvider.");
  }
  return value;
}
