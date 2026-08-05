import { QueryClientProvider, type QueryClient } from "@tanstack/react-query";
import { RouterProvider, type RouterProviderProps } from "react-router-dom";

import { ThemeProvider } from "./theme";

interface AppProvidersProps {
  queryClient: QueryClient;
  router: RouterProviderProps["router"];
}

export function AppProviders({ queryClient, router }: AppProvidersProps) {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <RouterProvider router={router} />
      </ThemeProvider>
    </QueryClientProvider>
  );
}
