export { type Config, CONFIG_VERSION, configSchema } from "./config";
export { parseConfig, type ParseConfigResult } from "./parse-config";
export {
  type Action,
  actionSchema,
  DEFAULT_RING,
  MAX_LABEL_CHARS,
  MAX_SLOTS,
  MIN_SLOTS,
  type Ring,
  ringSchema,
  type Slot,
  slotSchema,
} from "./ring";
export { parseShortcut, type ParseShortcutResult, shortcutSchema } from "./shortcut";
export {
  DEFAULT_TRIGGER,
  type KeyboardTrigger,
  keyboardTriggerSchema,
  type Modifier,
  modifierSchema,
  type MouseButton,
  mouseButtonSchema,
  type MouseTrigger,
  mouseTriggerSchema,
  type Trigger,
  triggerSchema,
} from "./trigger";
