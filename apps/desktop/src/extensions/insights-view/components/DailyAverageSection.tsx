import SectionContainer from "./SectionContainer";

const DailyAverageSection = () => {
  return (
    <SectionContainer loading={false}>
      <div className="flex flex-col space-y-2 items-start ml-2 p-3">
        <h3 className="font-semibold text-neutral-800">Daily Average</h3>
        <p className="text-neutral-700">2hr 15min per per day</p>
      </div>
    </SectionContainer>
  );
};

export default DailyAverageSection;
