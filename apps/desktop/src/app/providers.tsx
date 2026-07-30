import { QueryClientProvider, type QueryClient } from "@tanstack/react-query";
import { RouterProvider, type RouterProviderProps } from "react-router-dom";

interface AppProvidersProps {
  queryClient: QueryClient;
  router: RouterProviderProps["router"];
}

export function AppProviders({ queryClient, router }: AppProvidersProps) {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
