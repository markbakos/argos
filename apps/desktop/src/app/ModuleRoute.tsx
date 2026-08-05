import { Suspense } from "react";
import { Navigate, useLocation } from "react-router-dom";

import { getModuleRegistration } from "../modules/registry";
import { useModules } from "./modules";

export function ModuleRoute() {
  const route = useLocation().pathname;
  const modules = useModules();

  if (modules.isPending) {
    return <p role="status">Loading module…</p>;
  }
  if (modules.isError) {
    return (
      <section>
        <h1>Modules unavailable</h1>
        <p>Restart Argos or try opening this page again.</p>
      </section>
    );
  }
  const module = modules.data.modules.find(
    (candidate) => candidate.manifest.route === route,
  );
  if (!module) {
    return <Navigate to="/" replace />;
  }
  if (module.enablement === "disabled") {
    return <Navigate to="/settings" replace />;
  }
  const registration = getModuleRegistration(module.manifest.id);
  if (!registration) {
    return (
      <section>
        <h1>Module registration error</h1>
        <p>This module is missing its local interface.</p>
      </section>
    );
  }
  const Page = registration.page;
  return (
    <Suspense fallback={<p role="status">Loading module…</p>}>
      <Page module={module} />
    </Suspense>
  );
}
