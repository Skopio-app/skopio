import { Skeleton } from "@skopio/ui";
import { formatDuration } from "../../../utils/time";

interface ItemsListProps {
  title: string;
  data: {
    name: string;
    value: number;
  }[];
  loading: boolean;
}

const ItemsList: React.FC<ItemsListProps> = ({ title, data, loading }) => {
  return (
    <div className="flex flex-col items-center">
      <div className="flex flex-col space-y-2 w-fit">
        <h3 className="font-semibold text-center">{title}</h3>
        {data.length === 0 && !loading && (
          <p className="text-sm font-light">{`No ${title.toLowerCase()} found`}</p>
        )}
        {!loading && data ? (
          data.map((item) => (
            <div key={item.name} className="flex items-center gap-x-2">
              <p className="text-xs font-light w-20 text-right">
                {formatDuration(item.value)}
              </p>
              <p className="text-sm">{item.name}</p>
            </div>
          ))
        ) : (
          <Skeleton className="w-xl" />
        )}
      </div>
    </div>
  );
};

export default ItemsList;
