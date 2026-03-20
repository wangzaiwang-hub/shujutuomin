import FullWindowBrowser from "@/components/browser/FullWindowBrowser";

export default function CheersAICloudBrowser() {
  const cloudUrl = "https://7smile.dlithink.com/cheersai_desktop/apps/";

  return (
    <FullWindowBrowser
      initialUrl={cloudUrl}
    />
  );
}