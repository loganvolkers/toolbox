import seikika from "../fixtures/s4s-assessments.json";
import openfga from "../fixtures/openfga.json";
import { fixture, page } from "./report";

export default { title: "Pages/Assessments" };

const data = fixture(seikika);
const id = "assessment:crate-design/crate:seikika-expr";
const rubric = "rubric:seikika/assessment-quality";
const fields = data.entities![id].fields;
const timed = {
  ...data,
  entities: {
    ...data.entities,
    [id]: { ...data.entities![id], fields: { ...fields, at: "2026-11-27T21:42:00+01:00" } },
  },
};

export const RecentRuns = page(data, "assessments");
export const Overview = page(data, "overview");
export const RunDetail = page(data, "assessment/" + encodeURIComponent(id));
export const ReviewDetail = page(data, "assessment/" + encodeURIComponent("assessment:assessment-quality/" + id));
export const TimedRun = page(timed, "assessments");
export const TimedRunDetail = page(timed, "assessment/" + encodeURIComponent(id));
export const Rubrics = page(data, "rubrics");
export const RubricDetail = page(data, "rubric/" + encodeURIComponent(rubric));
export const NoAssessments = page(fixture(openfga), "assessments");
export const MissingRun = page(data, "assessment/missing");
