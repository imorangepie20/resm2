import { PropertyForm } from "./property-form"

export function NewPropertyPage() {
  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold">매물 등록</h1>
      <PropertyForm />
    </div>
  )
}
