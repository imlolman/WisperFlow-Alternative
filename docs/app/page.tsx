import { BackgroundGlow } from "@/components/BackgroundGlow";
import { FAQ } from "@/components/FAQ";
import { Footer } from "@/components/Footer";
import { Gallery } from "@/components/Gallery";
import { Header } from "@/components/Header";
import { MacGatekeeperTip } from "@/components/MacGatekeeperTip";
import { Hero } from "@/components/Hero";
import { Showcase } from "@/components/Showcase";
import { UseCases } from "@/components/UseCases";

export default function Home() {
  return (
    <>
      <BackgroundGlow />
      <Header />
      <main>
        <Hero />
        <Showcase />
        <UseCases />
        <Gallery />
        <MacGatekeeperTip />
        <FAQ />
      </main>
      <Footer />
    </>
  );
}
