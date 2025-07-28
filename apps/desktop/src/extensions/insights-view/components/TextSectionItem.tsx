import SectionContainer from "./SectionContainer";

interface TextSectionItemProps {
  title: string;
  text: string;
  loading: boolean;
}

const TextSectionItem: React.FC<TextSectionItemProps> = ({
  title,
  text,
  loading,
}) => {
  return (
    <SectionContainer loading={loading}>
      <div className="flex flex-col space-y-2 items-start ml-2 p-3">
        <h3 className="font-semibold text-neutral-800">{title}</h3>
        <p className="text-neutral-700">{text}</p>
      </div>
    </SectionContainer>
  );
};

export default TextSectionItem;
