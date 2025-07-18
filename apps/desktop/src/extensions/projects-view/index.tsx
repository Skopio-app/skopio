import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  Skeleton,
} from "@skopio/ui";
import { useEffect, useState } from "react";
import { commands, PaginatedProjects, Project } from "../../types/tauri.gen";

const ProjectsView = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [cursors, setCursors] = useState<(number | null)[]>([]);
  const [totalPages, setTotalPages] = useState<number>(0);
  const limit = 10;

  const fetchData = async (page: number) => {
    setIsLoading(true);
    try {
      const after = cursors[page] ?? null;
      const res: PaginatedProjects = await commands.fetchProjects({
        after,
        limit,
      });

      setProjects(res.data);
      setCursors(res.cursors ?? null);
      setTotalPages(res.totalPages ?? 0);
      console.log("The total pages: ", totalPages);
      console.log("The cursors: ", res.cursors);
    } catch (err) {
      console.error("Failed to fetch projects", err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData(currentPage);
  }, [currentPage]);

  const pageWindowSize = 5;
  const total = totalPages;
  const start = Math.max(0, currentPage - Math.floor(pageWindowSize / 2));
  const end = Math.min(total, start + pageWindowSize);

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto space-y-6">
        <ul className="divide-y divide-muted border">
          {isLoading
            ? Array.from({ length: limit }).map((_, i) => (
                <li key={i} className="p-4">
                  <Skeleton className="h-6 w-1/2 mb-2" />
                  <Skeleton className="h-4 w-1/3" />
                </li>
              ))
            : projects.map((project) => (
                <li
                  key={project.id}
                  className="p-4 hover:bg-muted/40 transition-colors"
                >
                  <div className="flex flex-col">
                    <h3 className="text-base font-medium break-words">
                      {project.name}
                    </h3>
                    <p className="text-sm text-muted-foreground break-all line-clamp-2">
                      {project.root_path || "No root path specified"}
                    </p>
                  </div>
                </li>
              ))}
        </ul>
      </div>

      {totalPages > 1 && (
        <div className="pt-4 mb-6">
          <Pagination className="mt-auto">
            <PaginationContent>
              <PaginationItem>
                <PaginationPrevious
                  onClick={() =>
                    setCurrentPage((prev) => Math.max(prev - 1, 0))
                  }
                />
              </PaginationItem>

              {start > 0 && (
                <>
                  <PaginationItem>
                    <PaginationLink onClick={() => setCurrentPage(0)}>
                      1
                    </PaginationLink>
                  </PaginationItem>
                  {start > 1 && (
                    <PaginationItem>
                      <span className="px-2">...</span>
                    </PaginationItem>
                  )}
                </>
              )}

              {Array.from({ length: end - start }, (_, i) => {
                const pageIndex = start + i;
                return (
                  <PaginationItem key={pageIndex}>
                    <PaginationLink
                      isActive={pageIndex === currentPage}
                      onClick={() => setCurrentPage(pageIndex)}
                    >
                      {pageIndex + 1}
                    </PaginationLink>
                  </PaginationItem>
                );
              })}

              {end < total && (
                <>
                  {end < total - 1 && (
                    <PaginationItem>
                      <span className="px-2">...</span>
                    </PaginationItem>
                  )}
                  <PaginationItem>
                    <PaginationLink onClick={() => setCurrentPage(total - 1)}>
                      {total}
                    </PaginationLink>
                  </PaginationItem>
                </>
              )}

              <PaginationItem>
                <PaginationNext
                  onClick={() =>
                    setCurrentPage((prev) =>
                      prev + 1 < totalPages ? prev + 1 : prev,
                    )
                  }
                />
              </PaginationItem>
            </PaginationContent>
          </Pagination>
        </div>
      )}
    </div>
  );
};

export default ProjectsView;
