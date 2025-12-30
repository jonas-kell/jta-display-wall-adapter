import { computed } from "vue";
import { numberFromRaceTime } from "./representation";

export enum RunPossibilities {
    Run15_1 = "Run15_1",
    Run15_2 = "Run15_2",
    Run20_1 = "Run20_1",
    Run20_2 = "Run20_2",
    Run30_1 = "Run30_1",
    Run30_2 = "Run30_2",
}

function possibilityFromNumberAndIndex(d: number, index: number): RunPossibilities | null {
    switch (index) {
        case 1:
            switch (d) {
                case 15:
                    return RunPossibilities.Run15_1;
                case 20:
                    return RunPossibilities.Run20_1;
                case 30:
                    return RunPossibilities.Run30_1;
            }
            break;
        case 2:
            switch (d) {
                case 15:
                    return RunPossibilities.Run15_2;
                case 20:
                    return RunPossibilities.Run20_2;
                case 30:
                    return RunPossibilities.Run30_2;
            }
            break;
    }

    return null;
}
import useMainStore from "../stores/main";
import { ApplicationMode, Athlete } from "./interfaceShared";
import { HeatCompetitorResult } from "./interfaceInbound";

const MAIN_HEAT_KEY = "THIS_IS_THE_MAIN_HEAT";

export function sharedAthleteFunctionality() {
    const mainStore = useMainStore();

    const athletesArray = computed(() => {
        return mainStore.athletesData;
    });

    // SPK stuff
    const finishTimes = computed(() => {
        let res: { [key: string]: { [key in RunPossibilities]: number | null } } = {};
        athletesArray.value.forEach((athlete) => {
            let data = {
                [RunPossibilities.Run15_1]: null as number | null,
                [RunPossibilities.Run15_2]: null as number | null,
                [RunPossibilities.Run20_1]: null as number | null,
                [RunPossibilities.Run20_2]: null as number | null,
                [RunPossibilities.Run30_1]: null as number | null,
                [RunPossibilities.Run30_2]: null as number | null,
            };

            athlete.heats_from_assignments.forEach((heat) => {
                const poss = possibilityFromNumberAndIndex(heat[1].distance, heat[1].heat_descriminator);

                if (poss) {
                    const heatCompetitorResult = heat[0];
                    if (heatCompetitorResult) {
                        data[poss] = numberFromRaceTime(heatCompetitorResult.runtime_full_precision);
                    }
                }
            });

            res[athlete.athlete.id] = data;
        });

        return res;
    });

    // Street run stuff
    const evaluations = computed(() => {
        if ((mainStore.staticConfiguration?.mode ?? ApplicationMode.SprinterKing) == ApplicationMode.StreetLongRun) {
            let res = {} as { [key: string]: { athlete: Athlete; evaluations: HeatCompetitorResult[] } };

            mainStore.athletesData.forEach((a) => {
                let evals = [] as HeatCompetitorResult[];

                if (mainStore.mainHeat) {
                    const heatData = mainStore.mainHeat;
                    if (heatData.meta.name == MAIN_HEAT_KEY) {
                        heatData.evaluations?.forEach((evaluation) => {
                            if (evaluation.competitor_result.competitor.bib == a.athlete.bib) {
                                evals.push(evaluation.competitor_result);
                            }
                        });
                    }
                }

                evals.sort((a, b) => {
                    return numberFromRaceTime(a.runtime_full_precision) - numberFromRaceTime(b.runtime_full_precision);
                });

                res[a.athlete.id] = {
                    athlete: a.athlete,
                    evaluations: evals,
                };
            });

            return res;
        } else {
            return {};
        }
    });

    return {
        athletesArray,
        finishTimes,
        evaluations,
    };
}
