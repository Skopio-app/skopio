import { Outlet } from "react-router";
import { Toaster } from "sonner";

function App() {
  return (
    <div>
      <Toaster richColors />
      <Outlet />
    </div>
  );
}

export default App;
