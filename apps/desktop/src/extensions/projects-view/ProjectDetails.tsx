import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { commands, Project, ProjectQuery } from "../../types/tauri.gen";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbSeparator,
  Skeleton,
} from "@skopio/ui";

const ProjectDetails = () => {
  const { projectId } = useParams();
  const [project, setProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchProject = async () => {
      try {
        if (!projectId) return;
        console.log("The project id: ", projectId);
        const query: ProjectQuery = {
          id: Number(projectId),
        };

        const result = await commands.fetchProject(query);
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
    return <p className="p-4 text-red-500">Project not found.</p>;
  }
  return (
    <div className="p-4 space-y-6">
      <Breadcrumb>
        <BreadcrumbList className="text-lg">
          <BreadcrumbItem>
            <BreadcrumbLink asChild>
              <BreadcrumbLink href="/">Projects</BreadcrumbLink>
            </BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbLink asChild>
              <BreadcrumbLink>{project.name}</BreadcrumbLink>
            </BreadcrumbLink>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      <h1 className="text-2xl font-semibold">{project.name}</h1>

      <p className="text-muted-foreground">
        Root path: {project.root_path ?? "N/A"}
      </p>
    </div>
  );
};

export default ProjectDetails;
