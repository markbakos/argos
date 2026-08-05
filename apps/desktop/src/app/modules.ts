import { useQuery } from "@tanstack/react-query";

import { api } from "../api";

export const modulesQueryKey = ["core", "modules"] as const;

export function useModules() {
  return useQuery({
    queryKey: modulesQueryKey,
    queryFn: () => api.core.listModules(),
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
    refetchOnWindowFocus: false,
  });
}
