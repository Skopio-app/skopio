export type PieChartData = {
  id: string;
  label: string;
  value: number;
};

export type BarChartData = {
  label: string;
  [value: string]: number | string;
};
