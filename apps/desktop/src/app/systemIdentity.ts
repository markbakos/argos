import { useQuery } from "@tanstack/react-query";

import { api } from "../api";

const SYSTEM_IDENTITY_QUERY_KEY = ["core", "system-identity"] as const;

export function useSystemIdentity() {
  return useQuery({
    queryKey: SYSTEM_IDENTITY_QUERY_KEY,
    queryFn: () => api.core.getSystemIdentity(),
    gcTime: Infinity,
    staleTime: Infinity,
    retry: false,
    retryOnMount: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });
}
