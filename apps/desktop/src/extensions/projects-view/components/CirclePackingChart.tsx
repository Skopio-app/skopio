import {
  CirclePackingCustomLayerProps,
  ResponsiveCirclePacking,
} from "@nivo/circle-packing";
import { formatDuration } from "../../../utils/time";
import { useState } from "react";

type CirclePackingNode =
  | { name: string; value: number }
  | { name: string; children: CirclePackingNode[] };

interface CirclePackingChartProps {
  data: { name: string; value: number }[];
}

const CirclePackingChart: React.FC<CirclePackingChartProps> = ({ data }) => {
  const [zoomedId, setZoomedId] = useState<string | null>(null);

  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  const packedData: CirclePackingNode = {
    name: "Total",
    children: [...data].sort((a, b) => b.value - a.value),
  };

  return (
    <div className="h-[800px] w-full">
      <ResponsiveCirclePacking<CirclePackingNode>
        data={packedData}
        zoomedId={zoomedId}
        onClick={(node) => setZoomedId(node.id)}
        margin={{ top: 20, right: 20, bottom: 20, left: 20 }}
        id="name"
        value="value"
        colors={{ scheme: "nivo" }}
        colorBy="id"
        padding={4}
        enableLabels={true}
        label="name"
        labelsSkipRadius={12}
        labelTextColor={{
          from: "color",
          modifiers: [["darker", 2]],
        }}
        borderWidth={1}
        borderColor={{ from: "color", modifiers: [["darker", 0.5]] }}
        tooltip={({ id, value }) => {
          const formattedTime = formatDuration(value);

          if (zoomedId === null || zoomedId !== "Total") {
            return <></>;
          }

          return (
            <div className="min-w-32 rounded-md border border-gray-200 bg-white px-3 py-2 text-sm shadow-md text-neutral-700">
              <h3 className="font-medium text-xs">{id}</h3>
              <p className="text-xs">{formattedTime}</p>
            </div>
          );
        }}
        layers={[
          "circles",
          ({ nodes }: CirclePackingCustomLayerProps<CirclePackingNode>) =>
            nodes
              .filter((node) => {
                if (node.height === 1) return false;
                if (zoomedId === "Total" || zoomedId === null) return true;
                return node.id === zoomedId;
              })
              .map((node) => {
                const { x, y, id, value } = node;

                const isZoomed = zoomedId !== null && zoomedId !== "Total";
                const minFont = isZoomed ? 12 : 6;
                const maxFont = isZoomed ? 16 : 14;
                const maxValue = Math.max(...nodes.map((n) => n.value));
                const fontSize =
                  minFont + (value / maxValue) * (maxFont - minFont);

                return (
                  <g key={id} transform={`translate(${x}, ${y})`}>
                    <text
                      textAnchor="middle"
                      dominantBaseline="middle"
                      fill="#000"
                      fontSize={fontSize}
                      fontWeight={600}
                      y={-6}
                    >
                      {id}
                    </text>
                    <text
                      textAnchor="middle"
                      dominantBaseline="middle"
                      fill="#555"
                      fontSize={fontSize * 0.8}
                      y={fontSize / 2}
                    >
                      {formatDuration(value)}
                    </text>
                  </g>
                );
              }),
        ]}
      />
    </div>
  );
};

export default CirclePackingChart;
