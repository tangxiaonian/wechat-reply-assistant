export type Category =
  "relationship" | "communication" | "preference" | "boundary" | "recent";
export type Tone = "自然" | "温柔" | "俏皮" | "正式";
export type Task = "reply" | "rewrite" | "follow_up";
export type ProfileTab = "info" | "observations" | "history";
export type SourceKind = "manual" | "chat_fact" | "user_confirmed";

export interface ChatMessage {
  id: string;
  direction: "self" | "other";
  text: string;
  at: string;
  senderId?: string;
  senderName?: string;
  replyToId?: string;
}

export interface Contact {
  kind?: "private" | "group";
  members?: { id: string; name: string }[];
  id: string;
  name: string;
  initial: string;
  relationship: string;
  tint: "sage" | "sand" | "blue";
  description: string;
  messages: ChatMessage[];
}

export interface Evidence {
  messageId: string;
  quote: string;
  at: string;
}

export interface ProfileItem {
  id: string;
  category: Category;
  subjectKey: string;
  subjectLabel: string;
  content: string;
  sourceKind: SourceKind;
  locked: boolean;
  scope: "long_term" | "recent";
  expiresAtMs: number | null;
  revision: number;
  evidence: Evidence[];
}

export interface Proposal {
  id: string;
  kind: "observation" | "conflict";
  category: Category;
  subjectKey: string;
  subjectLabel: string;
  content: string;
  targetItemId: string | null;
  evidence: Evidence[];
}

export interface ProfileEvent {
  id: string;
  itemId: string | null;
  summary: string;
  before: string | null;
  after: string | null;
  createdAtMs: number;
}

export interface Exclusion {
  id: string;
  subjectKey: string;
  subjectLabel: string;
}
export interface ContactProfile {
  items: ProfileItem[];
  proposals: Proposal[];
  events: ProfileEvent[];
  exclusions: Exclusion[];
}
export interface Workspace {
  contacts: Contact[];
  profiles: Record<string, ContactProfile>;
}

export type ProfileAction =
  | { type: "save"; contactId: string; item: ProfileItem }
  | {
      type: "delete";
      contactId: string;
      itemId: string;
      excludeFuture: boolean;
    }
  | {
      type: "decide";
      contactId: string;
      proposalId: string;
      decision: "keep_existing" | "accept" | "accept_as_recent" | "ignore";
      expiresAtMs?: number;
    }
  | { type: "remove_exclusion"; contactId: string; exclusionId: string }
  | { type: "demo_update"; contactId: string };

export interface ProviderSummary {
  name: string;
  baseUrl: string;
  model: string;
  configured: boolean;
  consent: boolean;
}
export interface ProviderInput {
  name: string;
  baseUrl: string;
  model: string;
  apiKey: string;
  consent: boolean;
}
export interface ReplyInput {
  requestId: string;
  contactId: string;
  task: Task;
  tone: string;
  customTone: string;
  background: string;
  useProfile: boolean;
  manualTranscript: string | null;
  selectedMessageIds?: string[];
  replyTargetId?: string | null;
}
export interface ToneTemplate {
  id: string;
  name: string;
  instruction: string;
  enabled: boolean;
  baseTone?: Tone;
}
export interface ReplyResult {
  candidates: string[];
  mode: "model" | "demo";
}

export const categories: Record<Category, string> = {
  relationship: "关系与称呼",
  communication: "沟通习惯",
  preference: "明确偏好",
  boundary: "相处边界",
  recent: "近期重要事项",
};
export const tones: Tone[] = ["自然", "温柔", "俏皮", "正式"];
export interface SyncSettings {
  enabled: boolean;
  intervalSeconds: number;
}
