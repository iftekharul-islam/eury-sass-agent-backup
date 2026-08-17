const USER_IMAGE_REQUEST_RE =
  /\b(?:generate|create|make|draw|show)\b.{0,30}\b(?:image|picture|photo|illustration|chart|diagram|map|visual)\b|\b(?:image|picture|photo)\b.{0,20}\b(?:of|for)\b/i;

const MODEL_IMAGE_DENIAL_RE =
  /don't (currently )?have access|cannot (?:generate|create)|can't (?:generate|create)|no image.generation|not have.*image.generat|paste into any image generat|external (?:image )?generat|ready-to-use prompt you can paste/i;

export function isUserImageRequest(text: string): boolean {
  return USER_IMAGE_REQUEST_RE.test(text.trim());
}

export function isModelImageDenial(text: string): boolean {
  return MODEL_IMAGE_DENIAL_RE.test(text);
}
