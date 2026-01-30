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
    <SectionContainer loading={loading} variant="text">
      <div className="flex flex-col space-y-2 items-start ml-2 p-3">
        <h3 className="font-semibold text-foreground">{title}</h3>
        <p className="text-muted-foreground">{text}</p>
      </div>
    </SectionContainer>
  );
};

export default TextSectionItem;
