import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { commands, Project } from "@/types/tauri.gen";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
  Skeleton,
} from "@skopio/ui";
import { mapRangeToPreset } from "@/utils/time";
import { formatDuration } from "@/utils/time";
import RangeSelectionDialog from "@/components/RangeSelectionDialog";
import { usePresetFilter } from "./stores/usePresetFilter";
import BranchSelectionDialog from "./components/BranchSelectionDialog";
import { useTotalBucketedTime } from "./hooks/useTotalBucketedTime";
import LineChartSection from "./components/sections/LineChartSection";
import CategoryChartSection from "./components/sections/CategoryChartSection";
import CirclePackingChartSection from "./components/sections/CirclePackingChartSection";
import EntityList from "./components/sections/EntityList";
import BranchList from "./components/sections/BranchList";
import AppPieChartSection from "./components/sections/AppPieChartSection";
import LanguagePieChartSection from "./components/sections/LanguagePieChartSection";
import { useDateRangeParams } from "@/hooks/useDateRangesParams";

const ProjectDetails = () => {
  const { projectId } = useParams();
  const [project, setProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);

  const {
    selectedRange,
    setSelectedRange,
    isCustom,
    pendingStart,
    pendingEnd,
    setPendingStart,
    setPendingEnd,
    setCustomStart,
    setCustomEnd,
    startDate,
    endDate,
  } = useDateRangeParams();

  useEffect(() => {
    const next = mapRangeToPreset(selectedRange, startDate, endDate);
    const prev = usePresetFilter.getState().preset;
    if (JSON.stringify(prev) !== JSON.stringify(next)) {
      usePresetFilter.setState({ preset: next });
    }
  }, [selectedRange, startDate, endDate]);

  const { total, loading: timeLoading, hasBranchData } = useTotalBucketedTime();

  const formattedDuration = formatDuration(total);

  useEffect(() => {
    const fetchProject = async () => {
      try {
        if (!projectId) return;
        const result = await commands.fetchProject(projectId);
        usePresetFilter.setState({ project: result?.name });
        setProject(result);
      } catch (err) {
        console.error("Failed to fetch project", err);
      } finally {
        setLoading(false);
      }
    };

    fetchProject();
  }, [projectId]);

  if (loading) {
    return <Skeleton className="p-4" />;
  }

  if (!project) {
    return (
      <p className="py-30 text-destructive flex items-center justify-center">
        Project not found.
      </p>
    );
  }

  return (
    <div className="px-4 space-y-6">
      <Breadcrumb>
        <BreadcrumbList className="text-lg">
          <BreadcrumbItem>
            <BreadcrumbLink href="/">Projects</BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbLink>{project.name}</BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      {!timeLoading ? (
        <p className="text-muted-foreground">
          <span className="font-medium text-foreground">
            {formattedDuration}
          </span>{" "}
          for{" "}
          <RangeSelectionDialog
            selectedRange={selectedRange}
            setSelectedRange={setSelectedRange}
            pendingStart={pendingStart}
            setPendingStart={setPendingStart}
            setPendingEnd={setPendingEnd}
            pendingEnd={pendingEnd}
            setCustomStart={setCustomStart}
            setCustomEnd={setCustomEnd}
            isCustom={isCustom}
          />{" "}
          in{" "}
          <span className="text-foreground font-semibold">{project.name}</span>{" "}
          {hasBranchData && (
            <>
              under <BranchSelectionDialog /> branches
            </>
          )}
        </p>
      ) : (
        <Skeleton className="h-4 max-w-76" />
      )}

      <div className="flex flex-row space-x-2">
        <LineChartSection />
        <CategoryChartSection />
      </div>
      <div className="flex flex-row space-x-2">
        <AppPieChartSection />
        <LanguagePieChartSection />
      </div>
      <div className="flex items-center justify-center">
        <CirclePackingChartSection />
      </div>
      <div className="flex flex-row justify-between mx-3 mb-3">
        <EntityList />
        <BranchList />
      </div>
    </div>
  );
};

export default ProjectDetails;
