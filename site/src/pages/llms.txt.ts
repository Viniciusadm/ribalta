import type { APIRoute } from "astro";
import { llmsIndex, textResponse } from "../llms";

export const GET: APIRoute = async () => textResponse(await llmsIndex());
