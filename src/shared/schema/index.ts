export { type Config, CONFIG_VERSION, configSchema } from "./config";
export { parseConfig, type ParseConfigResult } from "./parse-config";
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
