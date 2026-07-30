import { AppProviders } from "./providers";
import { createAppQueryClient } from "./query";
import { createAppRouter } from "./router";

const queryClient = createAppQueryClient();
const router = createAppRouter();

export function App() {
  return <AppProviders queryClient={queryClient} router={router} />;
}
