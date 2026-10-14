import type { APIRoute } from "astro";
import { llmsFull, textResponse } from "../llms";

export const GET: APIRoute = async () => textResponse(await llmsFull());
