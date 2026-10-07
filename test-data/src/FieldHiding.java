class FieldHiding {
    static class Base {
        private int value = 1;
        private String name = "base";
        int shared = 10;

        int baseValue() {
            return value;
        }

        String baseName() {
            return name;
        }

        void setBaseValue(int value) {
            this.value = value;
        }
    }

    static class Derived extends Base {
        private int value = 2;
        private String name = "derived";
        int shared = 20;

        int derivedValue() {
            return value;
        }

        String derivedName() {
            return name;
        }

        void setDerivedValue(int value) {
            this.value = value;
        }
    }

    public static void main(String[] args) {
        Derived derived = new Derived();
        Base base = derived;

        System.out.println(derived.baseValue());
        System.out.println(derived.derivedValue());
        System.out.println(derived.baseName());
        System.out.println(derived.derivedName());
        System.out.println(base.shared);
        System.out.println(derived.shared);

        derived.setDerivedValue(3);
        System.out.println(derived.baseValue());
        System.out.println(derived.derivedValue());

        derived.setBaseValue(4);
        System.out.println(derived.baseValue());
        System.out.println(derived.derivedValue());

        base.shared = 11;
        System.out.println(base.shared);
        System.out.println(derived.shared);
    }
}
