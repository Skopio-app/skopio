import { PermissionRow } from "../PermissionRow";

const Permission = () => {
  return (
    <div className="space-y-3">
      <PermissionRow kind="accessibility" />
      <PermissionRow kind="inputMonitoring" />
    </div>
  );
};

export default Permission;
